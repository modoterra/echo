<?php

// PHP parses function declarations without executing their body at declaration time.
// Source: https://www.php.net/manual/en/functions.user-defined.php
function filter()
{
    echo "inside";
}

echo "after\n";
