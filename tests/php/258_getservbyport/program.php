<?php
echo "eighty_tcp:[" . getservbyport(80, "tcp") . "]\n";
echo "fifty_three_udp:[" . getservbyport(53, "udp") . "]\n";
echo "missing_type:[" . gettype(getservbyport(-1, "tcp")) . "]\n";
echo "exists:[" . function_exists("getservbyport") . "]\n";
