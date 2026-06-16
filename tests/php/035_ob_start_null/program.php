<?php
// null explicitly means no output callback.
ob_start(null);
echo "Hello";
ob_end_flush();
