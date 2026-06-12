use strict;
use warnings;
use Data::Dumper qw(Dumper);

main();
exit(0);

sub main {
    my $lang = 'en';
    my $pages = get_pages();
    #print Dumper $pages;
    create_yaml_config_file($lang, $pages);
    execute('cargo', 'run', '--', '--logfile', "all-$lang.log", '--log', 'debug', "examples/all-$lang.yaml");
    #, '--local', 'pages'
}

sub get_pages {
    opendir my $dh, 'pages' or die;
    my @pages = sort
        map { s/_/ /g; $_ }
        map { substr $_, 0, -5 }
        grep { /^[A-Za-z].*\.json/ }
        readdir $dh;
    closedir $dh;
    return \@pages;
}

sub create_yaml_config_file {
    my ($lang, $pages) = @_;

    open(my $fh, '<:utf8', 'skeleton.yaml') or die;

    my @content;
    for my $row (<$fh>) {
        $row =~ s/^  title: .*/  title: All $lang/;
        $row =~ s/^id: .*/id: all-$lang-fixed-id/;
        $row =~ s/^  language: .*/  language: $lang/;
        $row =~ s/^images: .*/images: true/;
        $row =~ s/^output-file: .*/output-file: all-$lang.epub/;
        $row =~ s/^resources: .*/resources: true/;
        $row =~ s/^links_to_pages: .*/links_to_pages: true/;
        push @content, $row;
        if ($row =~ /^articles:/) {
            for my $page (@$pages) {
                push @content, "  - $page\n";
            }
            last;
        }
    }

    open(my $out, '>:utf8', "examples/all-$lang.yaml") or die;
    print $out join "", @content;
    close $out;
}

sub execute {
    my @cmd = @_;
    my $exit_code = system(@cmd);

    if ($exit_code != 0) {
        die "Failed to execute @cmd (exit code: $exit_code)\n";
    }
}


