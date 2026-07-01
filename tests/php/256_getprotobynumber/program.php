<?php
echo "six:[" . getprotobynumber(6) . "]\n";
echo "seventeen:[" . getprotobynumber(17) . "]\n";
echo "missing_type:[" . gettype(getprotobynumber(-1)) . "]\n";
echo "exists:[" . function_exists("getprotobynumber") . "]\n";
