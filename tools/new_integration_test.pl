#!/usr/bin/perl
use strict;
use warnings;
use File::Spec;

main();
exit(0);


sub main {
    my $lang = shift @ARGV or die "Usage: $0 <lang> <page_name>\n";
    my $page = shift @ARGV or die "Usage: $0 <lang> <page_name>\n";

    # Replace spaces with underscores for the filename
    my $filename = $page;
    $filename =~ s/ /_/g;
    my $id = $filename;
    $id =~ s/_/-/g;

    download_json($lang, $page, $filename);

    open(my $fh, '<:utf8', 'skeleton.yaml') or die;

    my @content;
    for my $row (<$fh>) {
        $row =~ s/^  title: .*/  title: $page/;
        $row =~ s/^id: .*/id: $id-fixed-id/;
        $row =~ s/^  language: .*/  language: $lang/;
        $row =~ s/^output-file: .*/output-file: $filename.epub/;
        push @content, $row;
        if ($row =~ /^articles:/) {
            push @content, "  - $page\n\n";
            last;
        }
    }

    open(my $out, '>:utf8', "examples/$filename.yaml") or die;
    print $out join "", @content;
    close $out;

    execute("cargo", "run", "examples/$filename.yaml");

    mkdir "expected/$filename";
    chdir "expected/$filename";
    execute("unzip", "../../$filename.epub");
    chdir "../..";


    my $lower = lc $filename;
    open(my $test, '>>:utf8', "tests/books.rs") or die;
    print $test qq{\n};
    print $test qq{#[test]\n};
    print $test "fn generate_${lower}_book_from_local_page_dump() {\n";
    print $test qq{    assert_generated_book_matches_expected("$filename");\n};
    print $test "}\n\n";
    close $test;

}

sub download_json {
    my ($lang, $page, $filename) = @_;
    my $encoded_page = url_encode($page);
    my $url = "https://$lang.wikipedia.org/w/api.php?action=parse&prop=wikitext&redirects=true&format=json&page=$encoded_page";

    # Create pages directory if it doesn't exist
    if (!-d "pages") {
        mkdir "pages" or die "Could not create 'pages' directory: $!\n";
    }

    my $output_path = File::Spec->catfile("pages", "$filename.json");

    print "Fetching '$page' ($lang) from Wikipedia...\n";

    # Use curl to fetch the URL securely
    execute('curl', '-s', '-L', '-A', 'wikipedia-to-epub-fetcher/0.1.0 (contact: info@example.com)', $url, '-o', $output_path);
}

sub execute {
    my @cmd = @_;
    my $exit_code = system(@cmd);

    if ($exit_code != 0) {
        die "Failed to execute @cmd (exit code: $exit_code)\n";
    }
}

sub replace {
    my ($filename, $this, $that) = @_;
    execute('perl', '-i', '-p', '-e', "s/$this/$that/", "examples/$filename.yaml");
}

sub url_encode {
    my ($str) = @_;
    $str =~ s/([^A-Za-z0-9\-._~])/sprintf("%%%02X", ord($1))/eg;
    return $str;
}




