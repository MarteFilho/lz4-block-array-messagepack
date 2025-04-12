#ifndef LZ4_MESSAGEPACK_H
#define LZ4_MESSAGEPACK_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Process JSON input and return LZ4 compressed MessagePack
 * @param input_json JSON string to process
 * @return Pointer to the result string (must be freed with free_string)
 */
const char* process_lz4_messagepack(const char* input_json);

/**
 * Free memory allocated by process_lz4_messagepack
 * @param ptr Pointer to the string to free
 */
void free_string(char* ptr);

#ifdef __cplusplus
}
#endif

#endif // LZ4_MESSAGEPACK_H 