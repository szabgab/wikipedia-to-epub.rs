#!/usr/bin/perl
use strict;
use warnings;
use File::Spec;
use JSON::PP;

binmode(STDOUT, ':utf8');

my $target_template = shift or die "Usage: $0 <template_name>\n";

# Resolve pages directory in current directory or parent directory
my $pages_dir = File::Spec->catdir('pages');
if (!-d $pages_dir) {
    $pages_dir = File::Spec->catdir('..', 'pages');
}
if (!-d $pages_dir) {
    die "Could not find 'pages' directory in current or parent directory.\n";
}

opendir(my $dh, $pages_dir) or die "Cannot open directory $pages_dir: $!\n";
my @files = sort grep { /\.json$/ && -f File::Spec->catfile($pages_dir, $_) } readdir($dh);
closedir($dh);

my $json_parser = JSON::PP->new->utf8;

for my $file (@files) {
    my $filepath = File::Spec->catfile($pages_dir, $file);
    open(my $fh, '<:utf8', $filepath) or next;
    my $content = do { local $/; <$fh> };
    close($fh);

    my $data;
    eval {
        $data = $json_parser->decode($content);
    };
    if ($@) {
        warn "Failed to parse JSON in $file: $@\n";
        next;
    }

    my $wikitext = $data->{parse}{wikitext}{'*'} // '';

    # Extract all balanced {{ ... }} blocks using recursive regex
    my @templates = $wikitext =~ /(\{\{(?:[^{}]++|(?1))*\}\})/g;

    for my $tmpl (@templates) {
        # Check if the block starts with the specified template name case-insensitively
        if ($tmpl =~ /^\{\{\s*\Q$target_template\E\s*(?:\||\}\})/i) {
            print "$file: $tmpl\n";
        }
    }
}
