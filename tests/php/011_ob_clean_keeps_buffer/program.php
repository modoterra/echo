<?php
ob_start();
echo "discarded";
ob_clean();
echo "kept\n";
ob_end_flush();
