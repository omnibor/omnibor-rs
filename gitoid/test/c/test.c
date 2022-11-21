#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <inttypes.h>
#include "gitoid.h"

int main() {
    printf("-- TESTING gitoid_new_from_str function\n");
    GitOid new_from_str_gitoid = gitoid_new_from_str(HashAlgorithm_Sha1, ObjectType_Blob, "hello world"); 
    printf("new_from_str gitoid length %lu\n", new_from_str_gitoid.len);
    printf("new_from_str gitoid value %" PRIu8 "\n", new_from_str_gitoid.value[0]);
    printf("\n");

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
    size_t size = (sizeof byte_array / sizeof *byte_array);
    for (size_t count = 0; count < size; ++count) {
        sscanf(position, "%2hhx", &byte_array[count]);
        position += 2;
    }
    uint8_t byte_array_length = sizeof byte_array;

    printf("-- TESTING gitoid_new_from_bytes funtion\n");
    GitOid new_from_bytes_gitoid = gitoid_new_from_bytes(HashAlgorithm_Sha1, ObjectType_Blob, &byte_array_length, *byte_array);
    printf("new_from_bytes gitoid length %lu\n", new_from_bytes_gitoid.len);
    printf("new_from_bytes gitoid value %" PRIu8 "\n", new_from_bytes_gitoid.value[0]);
    printf("\n");

    printf("-- TESTING gitoid_new_from_url function\n");
    char *url = "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03";
    GitOid new_from_url_gitoid = gitoid_new_from_url(url);
    printf("gitoid_new_from_url gitoid length: %lu\n", new_from_url_gitoid.len);
    printf("gitoid_new_from_url gitoid value: %" PRIu8 "\n", new_from_url_gitoid.value[0]);
    printf("\n");

    printf("-- TESTING gitoid_get_url function\n");
    char *gitoid_url_string = gitoid_get_url(&new_from_url_gitoid);
    printf("gitoid_get_url: %s\n", gitoid_url_string);
    gitoid_str_free(gitoid_url_string);
    printf("\n");

    printf("-- TESTING gitoid_hash_algorithm_name\n");
    const char *hash_algorithm = gitoid_hash_algorithm_name(new_from_url_gitoid.hash_algorithm);
    printf("gitoid_hash_algorithm_name: %s\n", hash_algorithm);
    printf("\n");

    printf("-- TESTING gitoid_object_type_name\n");
    const char *object_type = gitoid_object_type_name(new_from_url_gitoid.object_type);
    printf("gitoid_object_type_name: %s\n", object_type);
    printf("\n");
}
