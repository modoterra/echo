<?php
$title = "Tom & Jerry";
$snippet = "<a href=\"/?q=Tom & Jerry\">Tom's link</a>";

echo "amp:[" . htmlspecialchars($title) . "]\n";
echo "tag:[" . htmlspecialchars($snippet) . "]\n";
echo "plain:[" . htmlspecialchars("already safe") . "]\n";
echo "exists:[" . function_exists("htmlspecialchars") . "]\n";
