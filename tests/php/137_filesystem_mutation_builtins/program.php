<?php
$base = "filesystem-mutation-work";
$nested = $base . "/cache/daily";
$marker = $nested . "/marker.txt";
$copy = $nested . "/marker-copy.txt";
$final = $nested . "/marker-final.txt";

if (is_file($final)) {
    $cleanupFinal = unlink($final);
}
if (is_file($copy)) {
    $cleanupCopy = unlink($copy);
}
if (is_file($marker)) {
    $cleanupMarker = unlink($marker);
}
if (is_dir($nested)) {
    $cleanupNested = rmdir($nested);
}
if (is_dir($base . "/cache")) {
    $cleanupCache = rmdir($base . "/cache");
}
if (is_dir($base)) {
    $cleanupBase = rmdir($base);
}

echo "mkdir-recursive:[" . mkdir($nested, 0755, true) . "]\n";
echo "touch-created:[" . touch($marker, 1700000000) . "]\n";
echo "mtime:[" . filemtime($marker) . "]\n";
echo "copy-created:[" . copy($marker, $copy) . "]\n";
echo "rename-moved:[" . rename($copy, $final) . "]\n";
echo "copy-gone:[" . file_exists($copy) . "]\n";
echo "final-file:[" . is_file($final) . "]\n";
echo "unlink-final:[" . unlink($final) . "]\n";
echo "unlink-marker:[" . unlink($marker) . "]\n";
echo "rmdir-nested:[" . rmdir($nested) . "]\n";
echo "nested-gone:[" . is_dir($nested) . "]\n";

$cleanupCache = rmdir($base . "/cache");
$cleanupBase = rmdir($base);
