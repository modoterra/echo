<?php
echo "abc_abd:[" . similar_text("abc", "abd") . "]\n";
echo "kitten_sitting:[" . similar_text("kitten", "sitting") . "]\n";
echo "empty:[" . similar_text("abc", "") . "]\n";
echo "exists:[" . function_exists("similar_text") . "]\n";
