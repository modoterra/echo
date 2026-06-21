<?php
// array_slice() extracts positional windows from an array.
// Source: https://www.php.net/manual/en/function.array-slice.php
// array_chunk() splits arrays into fixed-size groups.
// Source: https://www.php.net/manual/en/function.array-chunk.php
$row = ["id" => 101, "sku" => "A-42", 7 => "warehouse", "status" => "active", 8 => "late", "owner" => "maya"];
$slice = array_slice($row, 1, -1);
$keep = array_slice($row, -4, 3, true);
$chunks = array_chunk($row, 2);
$chunksKeep = array_chunk($row, 2, true);

echo "slice-keys:[" . implode(",", array_keys($slice)) . "]\n";
echo "slice-values:[" . implode("|", array_values($slice)) . "]\n";
echo "keep-keys:[" . implode(",", array_keys($keep)) . "]\n";
echo "keep-values:[" . implode("|", array_values($keep)) . "]\n";
echo "chunk-count:[" . count($chunks) . "]\n";
echo "chunk0-keys:[" . implode(",", array_keys($chunks[0])) . "]\n";
echo "chunk1-values:[" . implode("|", array_values($chunks[1])) . "]\n";
echo "keep-chunk1-keys:[" . implode(",", array_keys($chunksKeep[1])) . "]\n";
echo "keep-chunk2-values:[" . implode("|", array_values($chunksKeep[2])) . "]\n";
echo "exists:[" . function_exists("array_slice") . function_exists("array_chunk") . "]\n";
