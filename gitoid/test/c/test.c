#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <assert.h>
#include "gitoid.h"

#define LEN(arr) (sizeof(arr) / sizeof(arr[0]));
#define TEST(NAME) {.name = #NAME, .fn = NAME}

void test_gitoid_new_from_str() {
    const GitOidSha1Blob* gitoid = gitoid_sha1_blob_new_from_str("hello world");
    assert(gitoid != NULL);
    assert(gitoid_sha1_blob_hash_len() == 20);
    assert(gitoid_sha1_blob_get_hash_bytes(gitoid)[0] == 149);
    gitoid_sha1_blob_free(gitoid);
}

void test_gitoid_new_from_bytes() {
    unsigned char bytes[] = {0x00, 0x01, 0x02, 0x03,
                             0x04, 0x05, 0x06, 0x07,
                             0x08, 0x09, 0x0A, 0x0B,
                             0x0C, 0x0D, 0x0E, 0x0F};
    uint8_t bytes_len = LEN(bytes);

    const GitOidSha1Blob* gitoid = gitoid_sha1_blob_new_from_bytes(
        bytes,
        bytes_len
    );

    assert(gitoid != NULL);
    assert(gitoid_sha1_blob_hash_len() == 20);
    assert(gitoid_sha1_blob_get_hash_bytes(gitoid)[0] == 182);
    gitoid_sha1_blob_free(gitoid);
}

void test_gitoid_new_from_url() {
    char *url = "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03";
    const GitOidSha256Blob* gitoid = gitoid_sha256_blob_new_from_url(url);
    assert(gitoid != NULL);
    assert(gitoid_sha256_blob_hash_len() == 32);
    assert(gitoid_sha256_blob_get_hash_bytes(gitoid)[0] == 254);
    gitoid_sha256_blob_free(gitoid);
}

void test_gitoid_get_url() {
    char *url_in = "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03";
    const GitOidSha256Blob* gitoid = gitoid_sha256_blob_new_from_url(url_in);
    assert(gitoid != NULL);
    const char *url_out = gitoid_sha256_blob_get_url(gitoid);
    assert(strncmp(url_in, url_out, 83) == 0);
    gitoid_str_free(url_out);
    gitoid_sha256_blob_free(gitoid);
}

void test_gitoid_hash_algorithm_name() {
    const GitOidSha1Blob* gitoid = gitoid_sha1_blob_new_from_str("hello world");
    assert(gitoid != NULL);
    const char *hash_algorithm = gitoid_sha1_blob_hash_algorithm_name(gitoid);
    assert(strncmp(hash_algorithm, "sha1", 4) == 0);
    gitoid_sha1_blob_free(gitoid);
}

void test_gitoid_object_type_name() {
    const GitOidSha1Blob* gitoid = gitoid_sha1_blob_new_from_str("hello world");
    assert(gitoid != NULL);
    const char *object_type = gitoid_sha1_blob_object_type_name(gitoid);
    assert(strncmp(object_type, "blob", 4) == 0);
    gitoid_sha1_blob_free(gitoid);
}

void test_gitoid_validity() {
    char *validity_url = "gitoid:blob:sha000:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03";
    const GitOidSha1Blob* gitoid = gitoid_sha1_blob_new_from_url(validity_url);
    assert(gitoid == NULL);

    char *expected_msg = "string is not a valid GitOID URL";
    char error_msg[256];
    gitoid_get_error_message(error_msg, 256);
    assert(strncmp(error_msg, expected_msg, 32) == 0);
}

typedef void (*test_fn)();

typedef struct test {
    const char *name;
    test_fn fn;
} test_t;

int main() {
    setvbuf(stdout, NULL, _IONBF, 0);

    test_t tests[] = {
        TEST(test_gitoid_new_from_str),
        TEST(test_gitoid_new_from_bytes),
        TEST(test_gitoid_new_from_url),
        TEST(test_gitoid_get_url),
        TEST(test_gitoid_hash_algorithm_name),
        TEST(test_gitoid_object_type_name),
        TEST(test_gitoid_validity),
    };

    size_t n_tests = LEN(tests);

    for (size_t i = 0; i < n_tests; ++i) {
        test_t test = tests[i];
        printf("[%zu/%zu] TESTING: test_%s... ", i + 1, n_tests, test.name);
        test.fn();
        printf("PASSED\n");
    }
}
