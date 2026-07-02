<?php
echo "ipv4:" . bin2hex(inet_pton("127.0.0.1")) . "\n";
echo "ipv6:" . bin2hex(inet_pton("::1")) . "\n";
echo "roundtrip:" . inet_ntop(inet_pton("2001:db8::1")) . "\n";
echo "bad-pton:" . gettype(inet_pton("172.27.1.04")) . "\n";
echo "bad-ntop:" . gettype(inet_ntop("bad")) . "\n";
echo "exists:" . function_exists("inet_pton") . function_exists("inet_ntop") . "\n";
