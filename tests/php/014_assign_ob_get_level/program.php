<?php
ob_start();
$level = ob_get_level();
ob_end_clean();
echo $level, "\n";
