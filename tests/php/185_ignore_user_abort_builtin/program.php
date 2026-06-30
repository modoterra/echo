<?php
echo "initial: " . ignore_user_abort() . "\n";
echo "previous: " . ignore_user_abort(true) . "\n";
echo "current: " . ignore_user_abort() . "\n";
echo "previous: " . ignore_user_abort(false) . "\n";
echo "current: " . ignore_user_abort() . "\n";
