<?php
$path = sys_get_temp_dir() . "/echo_strip_whitespace_fixture.php";
$written = file_put_contents($path, "<?php\n// leading comment\n\$name = \"Ada // not a comment\"; /* inline */\necho  \$name  .  \"\\\\n\";\n# tail\n");
$stripped = php_strip_whitespace($path);
echo "[" . $stripped . "]\n";
$deleted = unlink($path);
echo "exists:" . function_exists("php_strip_whitespace") . "\n";
