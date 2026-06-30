<?php
header_remove();
echo "all headers removed\n";

header_remove("X-Test");
echo "named header removed\n";
