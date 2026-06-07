#!/usr/bin/perl
use strict;
use warnings;
use JSON::PP;

binmode(STDOUT, ':utf8');

main();

sub main {
    my $json_path = shift @ARGV or die "Usage: $0 <path_to_json_file>\n";
    if (!-f $json_path) {
        die "File not found: $json_path\n";
    }

    my $json_parser = JSON::PP->new->utf8;

    open(my $fh, '<:utf8', $json_path) or die "Could not open file '$json_path': $!\n";
    my $content = do { local $/; <$fh> };
    close($fh);

    my $data;
    eval {
        $data = $json_parser->decode($content);
    };
    if ($@) {
        die "Failed to parse JSON: $@\n";
    }

    my $wikitext = $data->{parse}{wikitext}{'*'};
    if (!defined $wikitext) {
        die "Could not find wikitext in JSON file. Expected path: parse -> wikitext -> *\n";
    }

    my $pos = 0;
    my $nodes = parse_nodes(\$wikitext, \$pos);

    print_nodes($nodes, 0);
}

sub parse_nodes {
    my ($text_ref, $pos_ref, $end_marker) = @_;
    my @nodes;
    my $current_text = "";

    while ($$pos_ref < length($$text_ref)) {
        # Check if we reached the end marker (e.g., "}}" or "|}")
        if (defined $end_marker && substr($$text_ref, $$pos_ref, length($end_marker)) eq $end_marker) {
            last;
        }

        # Check for template start
        if (substr($$text_ref, $$pos_ref, 2) eq '{{') {
            if ($current_text ne "") {
                push @nodes, { type => 'text', value => $current_text };
                $current_text = "";
            }
            $$pos_ref += 2; # consume '{{'
            push @nodes, parse_template($text_ref, $pos_ref);
        }
        # Check for table start
        elsif (substr($$text_ref, $$pos_ref, 2) eq '{|') {
            if ($current_text ne "") {
                push @nodes, { type => 'text', value => $current_text };
                $current_text = "";
            }
            $$pos_ref += 2; # consume '{|'
            push @nodes, parse_table($text_ref, $pos_ref);
        }
        else {
            $current_text .= substr($$text_ref, $$pos_ref, 1);
            $$pos_ref++;
        }
    }

    if ($current_text ne "") {
        push @nodes, { type => 'text', value => $current_text };
    }

    return \@nodes;
}

sub parse_template {
    my ($text_ref, $pos_ref) = @_;
    
    my $name_nodes = parse_until($text_ref, $pos_ref, qr/\||\}\}/);
    
    my @params;
    while ($$pos_ref < length($$text_ref)) {
        if (substr($$text_ref, $$pos_ref, 2) eq '}}') {
            $$pos_ref += 2; # consume '}}'
            last;
        }
        elsif (substr($$text_ref, $$pos_ref, 1) eq '|') {
            $$pos_ref++; # consume '|'
            my $param_nodes = parse_until($text_ref, $pos_ref, qr/\||\}\}/);
            push @params, $param_nodes;
        }
        else {
            $$pos_ref++;
        }
    }
    
    return {
        type => 'template',
        name => $name_nodes,
        params => \@params,
    };
}

sub parse_table {
    my ($text_ref, $pos_ref) = @_;
    
    my $content_nodes = parse_until($text_ref, $pos_ref, qr/\|\}/);
    
    if (substr($$text_ref, $$pos_ref, 2) eq '|}') {
        $$pos_ref += 2; # consume '|}'
    }
    
    return {
        type => 'table',
        content => $content_nodes,
    };
}

sub parse_until {
    my ($text_ref, $pos_ref, $stop_regex) = @_;
    my @nodes;
    my $current_text = "";
    
    while ($$pos_ref < length($$text_ref)) {
        my $rem = substr($$text_ref, $$pos_ref);
        if ($rem =~ /^($stop_regex)/) {
            last;
        }
        
        if (substr($$text_ref, $$pos_ref, 2) eq '{{') {
            if ($current_text ne "") {
                push @nodes, { type => 'text', value => $current_text };
                $current_text = "";
            }
            $$pos_ref += 2;
            push @nodes, parse_template($text_ref, $pos_ref);
        }
        elsif (substr($$text_ref, $$pos_ref, 2) eq '{|') {
            if ($current_text ne "") {
                push @nodes, { type => 'text', value => $current_text };
                $current_text = "";
            }
            $$pos_ref += 2;
            push @nodes, parse_table($text_ref, $pos_ref);
        }
        else {
            $current_text .= substr($$text_ref, $$pos_ref, 1);
            $$pos_ref++;
        }
    }
    
    if ($current_text ne "") {
        push @nodes, { type => 'text', value => $current_text };
    }
    
    return \@nodes;
}

sub split_param_key_val {
    my ($nodes) = @_;
    
    if (@$nodes && $nodes->[0]{type} eq 'text') {
        my $text = $nodes->[0]{value};
        my $eq_idx = index($text, '=');
        if ($eq_idx != -1) {
            my $key = substr($text, 0, $eq_idx);
            my $val_first_text = substr($text, $eq_idx + 1);
            
            my @val_nodes;
            if ($val_first_text ne "") {
                push @val_nodes, { type => 'text', value => $val_first_text };
            }
            push @val_nodes, @$nodes[1..$#$nodes];
            
            return ($key, \@val_nodes);
        }
    }
    
    return (undef, $nodes);
}

sub print_nodes {
    my ($nodes, $indent) = @_;
    
    for my $node (@$nodes) {
        if ($node->{type} eq 'template') {
            my $name = trim(nodes_to_plain_text($node->{name}));
            print "  " x $indent . "Template: $name\n";
            
            my $param_idx = 1;
            for my $param (@{$node->{params}}) {
                my ($key, $val_nodes) = split_param_key_val($param);
                
                my $key_str;
                if (defined $key) {
                    $key_str = trim($key);
                } else {
                    $key_str = $param_idx;
                    $param_idx++;
                }
                
                my $has_structure = has_nested_structure($val_nodes);
                
                if ($has_structure) {
                    print "  " x ($indent + 1) . "Parameter: $key_str =\n";
                    print_nodes($val_nodes, $indent + 2);
                } else {
                    my $val_str = trim(nodes_to_plain_text($val_nodes));
                    print "  " x ($indent + 1) . "Parameter: $key_str = $val_str\n";
                }
            }
        }
        elsif ($node->{type} eq 'table') {
            print "  " x $indent . "Table:\n";
            print_nodes($node->{content}, $indent + 1);
        }
    }
}

sub nodes_to_plain_text {
    my ($nodes) = @_;
    my $text = "";
    for my $node (@$nodes) {
        if ($node->{type} eq 'text') {
            $text .= $node->{value};
        }
        elsif ($node->{type} eq 'template') {
            my $name = trim(nodes_to_plain_text($node->{name}));
            $text .= "{{$name}}";
        }
        elsif ($node->{type} eq 'table') {
            $text .= "{|...|}";
        }
    }
    return $text;
}

sub has_nested_structure {
    my ($nodes) = @_;
    for my $node (@$nodes) {
        if ($node->{type} eq 'template' || $node->{type} eq 'table') {
            return 1;
        }
    }
    return 0;
}

sub trim {
    my ($str) = @_;
    $str =~ s/^\s+//;
    $str =~ s/\s+$//;
    return $str;
}
