#include <stdio.h>
#include <stdint.h>
#include <string.h>
#include <inttypes.h>
#include "../../gitoid.h"

typedef struct GitOid gitoid;

extern GitOid new_from_str(HashAlgorithm, ObjectType, const char *str);
extern GitOid new_from_bytes(HashAlgorithm, ObjectType, const uint8_t *content, uintptr_t content_len);
extern GitOid new_from_url(const char *str);

int main() {
    printf("testing GitOid new_from_str function\n");
    GitOid new_from_str_gitoid = new_from_str(HashAlgorithm_Sha1, ObjectType_Blob, "hello world"); 
    printf("new_from_str gitoid length %lu\n", new_from_str_gitoid.len);
    printf("new_from_str gitoid value %" PRIu8 "\n", new_from_str_gitoid.value[0]);


    // Section that creates the byte array was heavily inspired by
    // https://stackoverflow.com/a/3409211/2308264
    const char string[] = "hello_world", *position = string;
    unsigned char byte_array[12];

    // Does not do error checking, meant solely for test purposes
    for (size_t count = 0; count < sizeof byte_array/sizeof *byte_array; count++) {
        sscanf(position, "%2hhx", &byte_array[count]);
        position += 2;
    }

    uint8_t byte_array_length = sizeof byte_array;

    printf("testing GitOid new_from_bytes funtion\n");
    GitOid new_from_bytes_gitoid = new_from_bytes(HashAlgorithm_Sha1, ObjectType_Blob, &byte_array_length, *byte_array);

    printf("new_from_bytes gitoid length %lu\n", new_from_bytes_gitoid.len);
    printf("new_from_bytes gitoid value %" PRIu8 "\n", new_from_bytes_gitoid.value[0]);

    printf("testing GitOid new_from_url function\n");
    const char *url = "gitoid:blob:sha256:fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03";
    GitOid new_from_url_gitoid = new_from_url(url);
    printf("new_from_url gitoid length %lu\n", new_from_url_gitoid.len);
    printf("new_from_url gitoid value %" PRIu8 "\n", new_from_url_gitoid.value[0]);

    printf("testing gitoid_url function\n");
    const char *gitoid_url_string = gitoid_url(&new_from_url_gitoid);
    printf("gitoid_url %s\n", gitoid_url_string);

    printf("testing gitoid_hash_algorithm\n");
    const char *hash_algorithm = gitoid_hash_algorithm(&new_from_url_gitoid);
    printf("Hash Algorithm %s\n", hash_algorithm);
}