<?php
$base = "filesystem-link-work";
$targetName = "target.txt";
$target = $base . "/" . $targetName;
$current = $base . "/current.txt";
$backup = $base . "/backup.txt";

if (is_link($current)) {
    $cleanupCurrent = unlink($current);
}
if (is_file($current)) {
    $cleanupCurrentFile = unlink($current);
}
if (is_file($backup)) {
    $cleanupBackup = unlink($backup);
}
if (is_file($target)) {
    $cleanupTarget = unlink($target);
}
if (is_dir($base)) {
    $cleanupBase = rmdir($base);
}

echo "mkdir:[" . mkdir($base) . "]\n";
echo "write-target:[" . file_put_contents($target, "release=2026\n") . "]\n";
echo "symlink-created:[" . symlink($targetName, $current) . "]\n";
echo "is-symlink:[" . is_link($current) . "]\n";
echo "readlink:[" . readlink($current) . "]\n";
echo "hardlink-created:[" . link($target, $backup) . "]\n";
echo "hardlink-is-link:[" . is_link($backup) . "]\n";
echo "hardlink-exists:[" . file_exists($backup) . "]\n";
echo "backup-content:[" . file_get_contents($backup) . "]\n";

$cleanupCurrent = unlink($current);
$cleanupBackup = unlink($backup);
$cleanupTarget = unlink($target);
$cleanupBase = rmdir($base);
