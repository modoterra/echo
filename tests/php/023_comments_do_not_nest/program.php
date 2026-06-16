<?php
// PHP block comments do not nest: the first closing marker ends the comment.
/* first opener
/* nested text ends at the first closer */
// This echo is outside the comment and must run.
echo "kept\n";
