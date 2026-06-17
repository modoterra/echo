<?php

// PHP user-defined functions receive arguments by value by default.
// Source: https://www.php.net/manual/en/functions.arguments.php
function say($message)
{
    echo $message;
}

say("hello\n");
