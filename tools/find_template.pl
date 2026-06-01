#!/usr/bin/perl
use strict;
use warnings;
use File::Spec;
use JSON::PP;

binmode(STDOUT, ':utf8');

my $target_template = shift or die "Usage: $0 <template_name>\n";

# Resolve standard cache root based on target OS
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

sub find_json_files {
    my ($dir) = @_;
    return unless -d $dir;

    my @files;
    opendir(my $dh, $dir) or return;
    my @entries = grep { $_ ne '.' && $_ ne '..' } readdir($dh);
    closedir($dh);

    for my $entry (@entries) {
        my $path = File::Spec->catfile($dir, $entry);
        if (-d $path) {
            push @files, find_json_files($path);
        } elsif (-f $path && $path =~ /\.json$/) {
            push @files, $path;
        }
    }
    return @files;
}

my @all_files;

# 1. Gather files from local pages folder
my $local_pages_dir = File::Spec->catdir('pages');
if (!-d $local_pages_dir) {
    $local_pages_dir = File::Spec->catdir('..', 'pages');
}
if (-d $local_pages_dir) {
    push @all_files, find_json_files($local_pages_dir);
}

# 2. Gather files from central cache folder
if ($cache_root) {
    my $cache_pages_dir = File::Spec->catdir($cache_root, 'pages');
    if (-d $cache_pages_dir) {
        push @all_files, find_json_files($cache_pages_dir);
    }
}

# De-duplicate files
my %seen;
@all_files = sort grep { !$seen{$_}++ } @all_files;

if (!@all_files) {
    die "Could not find any JSON page dumps in local pages/ or central cache pages/.\n";
}

my $json_parser = JSON::PP->new->utf8;

for my $filepath (@all_files) {
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
