<?php
$encoded = "&lt;a href=&quot;/?q=Tom &amp; Jerry&quot;&gt;Tom&#039;s link&lt;/a&gt;";

echo "decoded:[" . htmlspecialchars_decode($encoded) . "]\n";
echo "amp:[" . htmlspecialchars_decode("Tom &amp; Jerry") . "]\n";
echo "unchanged:[" . htmlspecialchars_decode("&copy; stays named") . "]\n";
echo "exists:[" . function_exists("htmlspecialchars_decode") . "]\n";
