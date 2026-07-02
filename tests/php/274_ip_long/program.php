<?php
echo "loopback:" . ip2long("127.0.0.1") . "\n";
echo "private:" . ip2long("192.168.1.10") . "\n";
echo "invalid:" . ip2long("999.1.1.1") . "\n";
echo "long:" . long2ip(2130706433) . "\n";
echo "signed:" . long2ip(-1) . "\n";
echo "ip2long-exists:" . function_exists("ip2long") . "\n";
echo "long2ip-exists:" . function_exists("long2ip") . "\n";
