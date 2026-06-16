<?php

// PHP accepts callable names through ob_start(), but an undefined function cannot
// be used as an output callback. Echo must not crash while callback support is
// still being wired through EchoValue normalization.
// Source: https://www.php.net/manual/en/function.ob-start.php
ob_start("missing_function");
echo "after\n";
ob_end_flush();
