<?php
ob_start();
echo "A";
ob_start();
echo "discarded";
ob_end_clean();
echo "C\n";
ob_end_flush();
