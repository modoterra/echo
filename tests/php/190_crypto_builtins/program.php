<?php
$fixture_file = "crypto_fixture_input.txt";
$written = file_put_contents($fixture_file, "abc");

echo "crypt-len:[" . strlen(crypt("hunter2", "$2y$05$abcdefghijklmnopqrstuv")) . "]\n";
$md5 = crypt("hunter2", "$1$abc$");
$sha256 = crypt("hunter2", "$5$hello$");
$sha512 = crypt("hunter2", "$6$hello$");
$std_des = crypt("hunter2", "ab");
$ext_des = crypt("hunter2", "_Gl/.K0Ay");

echo "crypt-md5-len:[" . strlen($md5) . "]\n";
echo "crypt-md5-prefix:[" . (strpos($md5, "$1$") === 0) . "]\n";
echo "crypt-md5-stable:[" . (crypt("hunter2", $md5) === $md5) . "]\n";

echo "crypt-sha256-len:[" . strlen($sha256) . "]\n";
echo "crypt-sha256-prefix:[" . (strpos($sha256, "$5$") === 0) . "]\n";
echo "crypt-sha256-stable:[" . (crypt("hunter2", $sha256) === $sha256) . "]\n";

echo "crypt-sha512-len:[" . strlen($sha512) . "]\n";
echo "crypt-sha512-prefix:[" . (strpos($sha512, "$6$") === 0) . "]\n";
echo "crypt-sha512-stable:[" . (crypt("hunter2", $sha512) === $sha512) . "]\n";

echo "crypt-std-des-len:[" . strlen($std_des) . "]\n";
echo "crypt-std-des-stable:[" . (crypt("hunter2", $std_des) === $std_des) . "]\n";
echo "crypt-ext-des-len:[" . strlen($ext_des) . "]\n";
echo "crypt-ext-des-prefix:[" . (strpos($ext_des, "_") === 0) . "]\n";
echo "crypt-ext-des-stable:[" . (crypt("hunter2", $ext_des) === $ext_des) . "]\n";

$password_hash = password_hash("hunter2", PASSWORD_DEFAULT, null);
echo "password-hash-len:[" . strlen($password_hash) . "]\n";
echo "password-verify-ok:[" . password_verify("hunter2", $password_hash) . "]\n";
echo "password-verify-fail:[" . password_verify("wrong", $password_hash) . "]\n";
echo "password-needs-rehash:[" . password_needs_rehash($password_hash, PASSWORD_BCRYPT, null) . "]\n";
echo "password-algos-count:[" . count(password_algos()) . "]\n";
echo "password-cost:[" . PASSWORD_BCRYPT_DEFAULT_COST . "]\n";
echo "password-default:[" . PASSWORD_DEFAULT . "]\n";
echo "password-bcrypt:[" . PASSWORD_BCRYPT . "]\n";
echo "password-argon2i:[" . PASSWORD_ARGON2I . "]\n";
echo "password-argon2id:[" . PASSWORD_ARGON2ID . "]\n";
echo "hash-hmac-constant:[" . HASH_HMAC . "]\n";
echo "crypt-blowfish:[" . CRYPT_BLOWFISH . "]\n";
echo "crypt-sha256:[" . CRYPT_SHA256 . "]\n";
echo "crypt-md5:[" . CRYPT_MD5 . "]\n";
echo "crypt-std-des:[" . CRYPT_STD_DES . "]\n";
echo "crypt-ext-des:[" . CRYPT_EXT_DES . "]\n";
echo "crypt-sha512:[" . CRYPT_SHA512 . "]\n";

echo "md5-file:[" . md5_file($fixture_file) . "]\n";
echo "sha1-file:[" . sha1_file($fixture_file) . "]\n";
echo "md5-file-raw-len:[" . strlen(md5_file($fixture_file, true)) . "]\n";
echo "sha1-file-raw-len:[" . strlen(sha1_file($fixture_file, true)) . "]\n";

echo "hash:[" . hash("sha256", "abc") . "]\n";
echo "hash-file:[" . hash_file("sha256", $fixture_file) . "]\n";
echo "hash-equals-true:[" . hash_equals(hash("sha256", "abc"), hash("sha256", "abc")) . "]\n";
echo "hash-equals-false:[" . hash_equals("abc", "def") . "]\n";

$ctx = hash_init("sha256", 0, "");
$updated = hash_update($ctx, "hello");
echo "hash-update:[" . hash_final($ctx, false) . "]\n";

$ctx = hash_init("sha256", HASH_HMAC, "key");
$updated = hash_update($ctx, "abc");
echo "hash-init-hmac:[" . hash_final($ctx, false) . "]\n";

$ctx = hash_init("sha256", 0, "");
$updated = hash_update_file($ctx, $fixture_file);
echo "hash-update-file:[" . hash_final($ctx, false) . "]\n";

$ctx = hash_init("sha256", 0, "");
$updated = hash_update($ctx, "hello");
$ctx_copy = hash_copy($ctx);
$updated = hash_update($ctx_copy, "!");
echo "hash-copy:[" . hash_final($ctx, false) . "]\n";
echo "hash-copy-updated:[" . hash_final($ctx_copy, false) . "]\n";

echo "hash-hmac:[" . hash_hmac("sha256", "abc", "key") . "]\n";
echo "hash-hmac-file:[" . hash_hmac_file("sha256", $fixture_file, "key") . "]\n";
echo "hash-hmac-algos-count:[" . count(hash_hmac_algos()) . "]\n";
echo "hash-hkdf:[" . hash_hkdf("sha256", "input key material", 32, "", "", false) . "]\n";
echo "hash-pbkdf2:[" . hash_pbkdf2("sha256", "password", "salt", 1, 32, false) . "]\n";

$stream_ctx = hash_init("sha256", 0, "");
$stream_fp = fopen($fixture_file, "r");
echo "hash-update-stream-open:[";
if (is_resource($stream_fp)) {
    echo "1";
}
echo "]\n";
echo "hash-update-stream:[" . hash_update_stream($stream_ctx, $stream_fp) . "]\n";
echo "hash-update-stream-hash:[" . hash_final($stream_ctx, false) . "]\n";
fclose($stream_fp);

$stream_ctx = hash_init("sha256", 0, "");
$stream_fp = fopen($fixture_file, "r");
echo "hash-update-stream-limited:[" . hash_update_stream($stream_ctx, $stream_fp, 2) . "]\n";
echo "hash-update-stream-limited-hash:[" . hash_final($stream_ctx, false) . "]\n";
fclose($stream_fp);

$random_bytes = random_bytes(16);
echo "random-bytes-len:[" . strlen($random_bytes) . "]\n";
echo "random-int-edge:[" . random_int(7, 7) . "]\n";

$removed = unlink($fixture_file);
