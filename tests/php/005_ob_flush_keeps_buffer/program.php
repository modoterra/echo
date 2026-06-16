<?php
ob_start();
echo "x";
ob_flush();
echo "y\n";
ob_end_flush();
