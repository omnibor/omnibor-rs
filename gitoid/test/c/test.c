#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <assert.h>
#include <inttypes.h>
#include "gitoid.h"

#define LEN(arr) (sizeof(arr) / sizeof(arr[0]));

void test_gitoid_new_from_str() {
    GitOid gitoid = gitoid_new_from_str(HashAlgorithm_Sha1, ObjectType_Blob, "hello world"); 
    assert(gitoid.len == 20);
    assert(gitoid.value[0] == 149);
}

void test_gitoid_new_from_bytes() {
    // Section that creates the byte array was heavily inspired by [1].
    //
    // Does not do error checking, and is intended solely for test purposes.
    // The length of `byte_array` is equal to the length of `string` plus one,
    // to make space for the nul-terminator.
    //
    // [1]: https://stackoverflow.com/a/3409211/2308264
    const char *string = "hello_world";
    const char *position = string;
    unsigned char byte_array[12];
    size_t size = LEN(byte_array);
    for (size_t count = 0; count < size; ++count) {
        sscanf(position, "%2hhx", &byte_array[count]);
        position += 2;
    }
    uint8_t byte_array_length = sizeof byte_array;

    GitOid gitoid = gitoid_new_from_bytes(HashAlgorithm_Sha1, ObjectType_Blob, &byte_array_length, *byte_array);
    assert(gitoid.len == 20);
    assert(gitoid.value[0] == 130);
}

void test_gitoid_new_from_url() {
    char *url = "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03";
    GitOid gitoid = gitoid_new_from_url(url);
    assert(gitoid.len == 32);
    assert(gitoid.value[0] == 254);
}

void test_gitoid_get_url() {
    char *url_in = "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03";
    GitOid gitoid = gitoid_new_from_url(url_in);
    char *url_out = gitoid_get_url(&gitoid);
    assert(strncmp(url_in, url_out, 83) == 0);
    gitoid_str_free(url_out);
}

void test_gitoid_hash_algorithm_name() {
    GitOid gitoid = gitoid_new_from_str(HashAlgorithm_Sha1, ObjectType_Blob, "hello world");
    const char *hash_algorithm = gitoid_hash_algorithm_name(gitoid.hash_algorithm);
    assert(strncmp(hash_algorithm, "sha1", 4) == 0);
}

void test_gitoid_object_type_name() {
    GitOid gitoid = gitoid_new_from_str(HashAlgorithm_Sha1, ObjectType_Blob, "hello world");
    const char *object_type = gitoid_object_type_name(gitoid.object_type);
    assert(strncmp(object_type, "blob", 4) == 0);
}

void test_gitoid_validity() {
    // Notice the SHA type is invalid.
    char *validity_url = "gitoid:blob:sha000:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03";
    GitOid gitoid = gitoid_new_from_url(validity_url);
    assert(gitoid_invalid(&gitoid));
}

typedef void (*test_fn)();

typedef struct test {
    const char *name;
    test_fn fn;
} test_t;

int main() {
    setvbuf(stdout, NULL, _IONBF, 0);

    test_t tests[7] = {
        {.name = "gitoid_new_from_str", .fn = test_gitoid_new_from_str},
        {.name = "gitoid_new_from_bytes", .fn = test_gitoid_new_from_bytes},
        {.name = "gitoid_new_from_url", .fn = test_gitoid_new_from_url},
        {.name = "gitoid_get_url", .fn = test_gitoid_get_url},
        {.name = "gitoid_hash_algorithm_name", .fn = test_gitoid_hash_algorithm_name},
        {.name = "gitoid_object_type_name", .fn = test_gitoid_object_type_name},
        {.name = "gitoid_validity", .fn = test_gitoid_validity},
    };

    size_t n_tests = LEN(tests);

    for (size_t i = 0; i < n_tests; ++i) {
        test_t test = tests[i];
        printf("[%zu/%zu] TESTING: test_%s... ", i + 1, n_tests, test.name);
        test.fn();
        printf("PASSED\n");
    }
}
