<?php
ob_start();
echo "A";
ob_start();
echo "B";
ob_flush();
echo "C";
ob_end_flush();
echo "D\n";
ob_end_flush();
