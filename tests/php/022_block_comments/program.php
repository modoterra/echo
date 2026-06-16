<?php
// This echo runs before the block comment.
echo "A";
// Everything inside this C-style block comment should be ignored.
/*
echo "discarded";
*/
// Execution resumes after the first closing block-comment marker.
echo "B\n";
