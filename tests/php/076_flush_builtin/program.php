<?php
echo "A";
flush();

ob_start();
echo "B";
flush();
echo "C";
ob_end_flush();

echo "D";
flush();
