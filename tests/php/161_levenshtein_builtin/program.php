<?php
echo levenshtein("kitten", "sitting") . "\n";
echo levenshtein("flaw", "lawn") . "\n";
echo levenshtein("gumbo", "gambol") . "\n";
echo levenshtein("abc", "adc", 1, 5) . "\n";
echo levenshtein("", "echo") . "\n";
