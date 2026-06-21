<?php
$base = "temporary-name-work";

if (is_dir($base)) {
    $cleanupBase = rmdir($base);
}

echo "temp-dir-is-dir:[" . is_dir(sys_get_temp_dir()) . "]\n";
echo "mkdir:[" . mkdir($base) . "]\n";

$draft = tempnam($base, "exo");
echo "tempnam-string:[" . is_string($draft) . "]\n";
echo "tempnam-file:[" . is_file($draft) . "]\n";
echo "tempnam-prefix:[" . str_starts_with(basename($draft), "exo") . "]\n";

$job = uniqid("job_");
$entropy = uniqid("job_", true);
echo "uniqid-prefix:[" . str_starts_with($job, "job_") . "]\n";
echo "uniqid-length:[" . strlen(uniqid()) . "]\n";
echo "uniqid-entropy-length:[" . strlen($entropy) . "]\n";

$cleanupDraft = unlink($draft);
$cleanupBase = rmdir($base);
