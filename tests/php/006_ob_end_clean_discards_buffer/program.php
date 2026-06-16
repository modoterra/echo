<?php
ob_start();
echo "discarded";
ob_end_clean();
echo "kept\n";
