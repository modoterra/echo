<?php
// PHP 8.5 adds the pipe operator for left-to-right callable pipelines.
// Source: https://www.php.net/releases/8.5/en.php
echo " PHP 8.5 " |> trim(...) |> strtoupper(...);
echo "\n";
