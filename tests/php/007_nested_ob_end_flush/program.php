<?php
ob_start();
echo "A";
ob_start();
echo "B";
ob_end_flush();
echo "C\n";
ob_end_flush();
