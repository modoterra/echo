<?php
$html = "<p>Hello <strong>Ada</strong></p>";
$comment = "Keep<!-- hidden -->Visible";
$php = "before <?php echo 'secret'; ?> after";
$nul = "A" . chr(0) . "B";

echo "html:[" . strip_tags($html) . "]\n";
echo "comment:[" . strip_tags($comment) . "]\n";
echo "php:[" . strip_tags($php) . "]\n";
echo "nul:[" . strip_tags($nul) . "]\n";
echo "exists:[" . function_exists("strip_tags") . "]\n";
