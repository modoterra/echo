<?php

// The string concatenation operator converts operands to strings at runtime.
// Source: https://www.php.net/manual/en/language.operators.string.php
function greet($name)
{
    echo "Hello, " . $name . "!\n";
}

greet("Echo");
