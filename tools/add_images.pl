#!/usr/bin/perl
use strict;
use warnings;
use File::Path qw(make_path);
use JSON::PP qw(decode_json);
use utf8;

main();
exit(0);

sub main {
    my $config_file = shift @ARGV or die "Usage: $0 <config_file>\n";

    if (!-f $config_file) {
        die "Error: Configuration file '$config_file' not found.\n";
    }

    # Extract language from config file
    open my $fh, '<:utf8', $config_file or die "Can't read $config_file: $!\n";
    my $content = do { local $/; <$fh> };
    close $fh;

    my $language = 'en'; # default
    if ($content =~ /^\s*language:\s*['"]?([a-z-]+)['"]?/mi) {
        $language = $1;
    }

    print "Reading config file '$config_file' (language: $language)...\n";
    print "Running cargo to identify missing image fixtures...\n";

    # Run cargo run and parse stderr to identify missing image names
    my @missing_images;
    open my $cmd_fh, '-|', "cargo run -- $config_file --local pages 2>&1"
        or die "Failed to execute cargo run: $!\n";
    while (my $line = <$cmd_fh>) {
        $line =~ s/\x1B\[[0-9;]*[a-zA-Z]//g; # Strip ANSI escape sequences/colors
        if ($line =~ /omitting image image="([^"]+)"/) {
            push @missing_images, $1;
        }
    }
    close $cmd_fh;

    # Deduplicate the missing images
    my %seen;
    @missing_images = grep { !$seen{$_}++ } @missing_images;

    if (!@missing_images) {
        print "No missing image fixtures found.\n";
        return;
    }

    print "Found " . scalar(@missing_images) . " missing image(s):\n";
    for my $img (@missing_images) {
        print "  - $img\n";
    }

    # Load existing manifest if present
    my $manifest_path = "pages/images/manifest.json";
    my $manifest = {};
    if (-f $manifest_path) {
        open my $mfh, '<', $manifest_path or die "Can't read $manifest_path: $!\n";
        my $mcontent = do { local $/; <$mfh> };
        close $mfh;
        if ($mcontent =~ /\S/) {
            eval {
                $manifest = decode_json($mcontent);
            };
            if ($@) {
                warn "Warning: failed to parse existing manifest.json: $@\n";
            }
        }
    }

    make_path("pages/images");

    my $updated_count = 0;

    for my $image_name (@missing_images) {
        print "\nProcessing '$image_name'...\n";
        my $api_url = "https://$language.wikipedia.org/w/api.php?action=query&prop=imageinfo&iiprop=url|mime&iiurlwidth=800&format=json&titles=File:" . url_encode($image_name);

        # Fetch image metadata from Wikipedia API
        my $json_content = curl_get($api_url);
        if (!defined $json_content) {
            warn "Error: Failed to fetch metadata for '$image_name'\n";
            next;
        }

        my $data = eval { decode_json($json_content); };
        if ($@ || !$data) {
            warn "Error: Failed to parse metadata JSON for '$image_name'\n";
            next;
        }

        my $pages = $data->{query}{pages};
        if (!$pages) {
            warn "Error: No query pages returned for '$image_name'\n";
            next;
        }

        my ($page_id) = keys %$pages;
        my $imageinfo = $pages->{$page_id}{imageinfo}[0];
        if (!$imageinfo) {
            warn "Error: File '$image_name' does not exist on Wikipedia/Commons.\n";
            next;
        }

        my $download_url = $imageinfo->{thumburl} || $imageinfo->{url};

        if (!$download_url) {
            warn "Error: No download URL found for '$image_name'\n";
            next;
        }

        # Extract actual file extension from the download URL
        my $url_path = $download_url;
        $url_path =~ s/\?.*$//; # remove query parameters if any
        my $ext = 'img';
        if ($url_path =~ /\.([a-zA-Z0-9]+)$/) {
            $ext = lc($1);
        }
        if ($ext eq 'jpeg') {
            $ext = 'jpg';
        }

        # Determine correct media-type matching the downloaded file extension
        my %mime_map = (
            jpg  => 'image/jpeg',
            jpeg => 'image/jpeg',
            png  => 'image/png',
            gif  => 'image/gif',
            svg  => 'image/svg+xml',
            webp => 'image/webp',
        );
        my $media_type = $mime_map{$ext} || $imageinfo->{mime} || 'application/octet-stream';

        # Determine clean filename using the correct extension of the downloaded file
        my $base = $image_name;
        $base =~ s/\.[a-zA-Z0-9]+$//; # strip original extension
        $base =~ s/[^\p{L}\p{N}\-\.\(\)_]/_/g; # replace unsafe chars with underscore
        $base =~ s/__+/_/g;           # clean up multiple underscores

        my $filename = "${base}.${ext}";

        my $dest_path = "pages/images/$filename";
        print "Downloading from $download_url to $dest_path...\n";

        if (!curl_download($download_url, $dest_path)) {
            warn "Error: Failed to download image content\n";
            next;
        }

        # Update manifest entry
        $manifest->{$image_name} = {
            path => $filename,
            'media-type' => $media_type,
        };
        $updated_count++;

        # Rate-limiting request delay to avoid Wikipedia API abuse/throttling
        sleep(1);
    }

    if ($updated_count > 0) {
        # Save updated manifest
        my $json_encoder = JSON::PP->new->utf8->pretty->canonical;
        my $new_manifest_content = $json_encoder->encode($manifest);
        open my $mfh, '>', $manifest_path or die "Can't write to $manifest_path: $!\n";
        print $mfh $new_manifest_content;
        close $mfh;
        print "\nSuccessfully downloaded $updated_count image(s) and updated $manifest_path\n";
    } else {
        print "\nNo images were downloaded.\n";
    }
}

sub curl_get {
    my ($url) = @_;
    my $ua = 'wikipedia-to-epub/0.1.3 (https://github.com/szabgab/wikipedia-to-epub.rs; contact: https://github.com/szabgab/wikipedia-to-epub.rs/issues)';
    my @cmd = ('curl', '-s', '-L', '-A', $ua, $url);
    open my $cfh, '-|', @cmd or do {
        warn "Error: Failed to spawn curl: $!\n";
        return undef;
    };
    my $content = do { local $/; <$cfh> };
    close $cfh;
    if ($? != 0) {
        warn "Error: curl failed with exit code $?\n";
        return undef;
    }
    return $content;
}

sub curl_download {
    my ($url, $dest_path) = @_;
    my $ua = 'wikipedia-to-epub/0.1.3 (https://github.com/szabgab/wikipedia-to-epub.rs; contact: https://github.com/szabgab/wikipedia-to-epub.rs/issues)';
    system('curl', '-s', '-L', '-A', $ua, $url, '-o', $dest_path);
    if ($? != 0) {
        return 0;
    }
    return 1;
}

sub url_encode {
    my ($str) = @_;
    $str =~ s/([^A-Za-z0-9\-._~])/sprintf("%%%02X", ord($1))/eg;
    return $str;
}

sub extension_from_mime {
    my ($mime) = @_;
    $mime =~ s/;.*//; # remove parameters
    $mime = lc($mime);
    if ($mime eq 'image/jpeg') { return 'jpg'; }
    if ($mime eq 'image/png')  { return 'png'; }
    if ($mime eq 'image/gif')  { return 'gif'; }
    if ($mime eq 'image/svg+xml') { return 'svg'; }
    if ($mime eq 'image/webp') { return 'webp'; }
    return 'img';
}
