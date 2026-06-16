<?php
ob_start();
echo "A";
$value = ob_get_contents();
echo "B";
ob_end_clean();
echo $value, "\n";
