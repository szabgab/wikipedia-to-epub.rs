#!/usr/bin/perl
use strict;
use warnings;
use File::Spec;
use Data::Dumper qw(Dumper);
use JSON::PP;
use List::Util qw(uniq);

binmode(STDOUT, ':utf8');

main();

# Resolve standard cache root based on target OS

sub get_cache_root {
    my $cache_root;
    if ($^O eq 'MSWin32') {
        my $localappdata = $ENV{LOCALAPPDATA} || $ENV{APPDATA};
        $cache_root = File::Spec->catdir($localappdata, 'wikipedia-to-epub') if $localappdata;
    } elsif ($^O eq 'darwin') {
        my $home = $ENV{HOME};
        $cache_root = File::Spec->catdir($home, 'Library', 'Caches', 'wikipedia-to-epub') if $home;
    } else {
        my $cache_home = $ENV{XDG_CACHE_HOME};
        if ($cache_home) {
            $cache_root = File::Spec->catdir($cache_home, 'wikipedia-to-epub');
        } else {
            my $home = $ENV{HOME};
            $cache_root = File::Spec->catdir($home, '.cache', 'wikipedia-to-epub') if $home;
        }
    }
    return $cache_root;
}

sub find_json_files {
    my ($dir) = @_;
    return unless -d $dir;

    my @files;
    opendir(my $dh, $dir) or return;
    my @entries = grep { $_ ne '.' and $_ ne '..' } readdir($dh);
    closedir($dh);

    for my $entry (@entries) {
        my $path = File::Spec->catfile($dir, $entry);
        if (-d $path) {
            push @files, find_json_files($path);
        } elsif (-f $path and $path =~ /\.json$/ and $path !~ /manifest\.json$/) {
            push @files, $path;
        }
    }
    return @files;
}


sub find_matches {
    my ($target_template, $all_files) = @_;

    my $json_parser = JSON::PP->new->utf8;

    for my $filepath (@$all_files) {
        open(my $fh, '<:utf8', $filepath) or next;
        my $content = do { local $/; <$fh> };
        close($fh);

        my $data;
        eval {
            $data = $json_parser->decode($content);
        };
        if ($@) {
            warn "Failed to parse JSON in $filepath: $@\n";
            next;
        }

        my $wikitext = $data->{parse}{wikitext}{'*'} // '';

        # Extract all balanced {{ ... }} blocks using recursive regex
        my @templates = $wikitext =~ /(\{\{(?:[^{}]++|(?1))*\}\})/g;

        for my $tmpl (@templates) {
            # Check if the block starts with the specified template name case-insensitively
            if ($tmpl =~ /^\{\{\s*\Q$target_template\E\s*(?:\||\}\})/i) {
                print "$filepath: $tmpl\n";
            }
        }
    }
}

sub files_from_pages_folder {
    my @all_files;
    my $local_pages_dir = File::Spec->catdir('pages');
    #if (!-d $local_pages_dir) {
    #    $local_pages_dir = File::Spec->catdir('..', 'pages');
    #}
    if (-d $local_pages_dir) {
        push @all_files, find_json_files($local_pages_dir);
    }
    return uniq @all_files;
}

sub files_from_central_cache_folder {
    my $cache_root = get_cache_root();

    my @all_files;
    if ($cache_root) {
        my $cache_pages_dir = File::Spec->catdir($cache_root, 'pages');
        if (-d $cache_pages_dir) {
            push @all_files, find_json_files($cache_pages_dir);
        }
    }
    return uniq @all_files;
}


sub main {
    my $target_template = shift @ARGV or die "Usage: $0 <template_name>\n";

    my @local_files = files_from_pages_folder();
    printf("Local files (found %s files)\n", scalar @local_files);
    #die Dumper \@local_files;
    find_matches($target_template, \@local_files);

    my @cached_files = files_from_central_cache_folder();
    printf("Cached files (found %s files)\n", scalar @cached_files);
    find_matches($target_template, \@cached_files);
}




