<?php
// str_split() returns an array of byte chunks; chunk_split() inserts a
// separator after each byte chunk in a single string.
// Sources:
// - https://www.php.net/manual/en/function.str-split.php
// - https://www.php.net/manual/en/function.chunk-split.php
echo "split-default:[" . implode("|", str_split("Echo")) . "]\n";
echo "split-two:[" . implode("|", str_split("abcdef", 2)) . "]\n";
echo "split-uneven:[" . implode("|", str_split("abcde", 2)) . "]\n";
echo "split-empty:[" . implode("|", str_split("")) . "]\n";
echo "split-coerce:[" . implode("|", str_split(12345, "2")) . "]\n";
echo "chunk-two:[" . chunk_split("abcdef", 2, "|") . "]\n";
echo "chunk-uneven:[" . chunk_split("abcde", 2, "|") . "]\n";
echo "chunk-empty:[" . chunk_split("", 2, "|") . "]\n";
echo "chunk-coerce:[" . chunk_split(12345, "2", 0) . "]\n";
echo "exists:[" . function_exists("str_split") . function_exists("chunk_split") . "]\n";
