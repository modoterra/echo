<?php
echo "basic:[" . htmlentities("A & B <tag> \"q\"") . "]\n";
echo "exists:[" . function_exists("htmlentities") . "]\n";
