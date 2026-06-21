<?php
$base = "filesystem-content-work";
$report = $base . "/report.txt";

if (is_file($report)) {
    $cleanupReport = unlink($report);
}
if (is_dir($base)) {
    $cleanupBase = rmdir($base);
}

echo "mkdir:[" . mkdir($base) . "]\n";
echo "write-first:[" . file_put_contents($report, "alpha\nbeta\ngamma\n") . "]\n";
echo "append:[" . file_put_contents($report, "delta\n", 8) . "]\n";
echo "part:[" . file_get_contents($report, false, null, 6, 4) . "]\n";
echo "tail:[" . file_get_contents($report, false, null, -6, 5) . "]\n";
echo "stream:[";
$bytes = readfile($report);
echo "]\n";
echo "readfile-bytes:[" . $bytes . "]\n";

$cleanupReport = unlink($report);
$cleanupBase = rmdir($base);
