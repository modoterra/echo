<?php
// filemtime(), filectime(), fileatime(), fileinode(), fileowner(), filegroup(),
// and fileperms() expose local stat metadata as integers or false on failure.
// filetype() reports strings such as "file" and "dir".
// Sources: https://www.php.net/manual/en/function.filemtime.php and https://www.php.net/manual/en/function.filetype.php
$file = __DIR__ . "/program.php";
$dir = __DIR__;

echo "filetype-file:[" . filetype($file) . "]\n";
echo "filetype-dir:[" . filetype($dir) . "]\n";
echo "mtime-int:[" . is_int(filemtime($file)) . "]\n";
echo "ctime-int:[" . is_int(filectime($file)) . "]\n";
echo "atime-int:[" . is_int(fileatime($file)) . "]\n";
echo "inode-int:[" . is_int(fileinode($file)) . "]\n";
echo "owner-int:[" . is_int(fileowner($file)) . "]\n";
echo "group-int:[" . is_int(filegroup($file)) . "]\n";
echo "perms-int:[" . is_int(fileperms($file)) . "]\n";
echo "exists:[" . function_exists("filemtime") . function_exists("filectime") . function_exists("fileatime") . function_exists("fileinode") . function_exists("fileowner") . function_exists("filegroup") . function_exists("fileperms") . function_exists("filetype") . "]\n";
