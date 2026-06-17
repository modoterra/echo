<?php
echo "A";
ob_implicit_flush(true);

ob_start();
echo "B";
ob_implicit_flush(true);
echo "C";
ob_implicit_flush(false);
echo "D";
ob_end_flush();

echo "E";
ob_implicit_flush(false);
