# AgentMem C Library Makefile
# Mock implementation for Cangjie SDK testing

CC = gcc
CFLAGS = -Wall -Wextra -O2 -fPIC -std=c99
LDFLAGS = -shared

# Library name
LIB_NAME = libagentmem_c
STATIC_LIB = $(LIB_NAME).a
SHARED_LIB = $(LIB_NAME).so

# Source files
SOURCES = agentmem_c.c
OBJECTS = $(SOURCES:.c=.o)
HEADERS = agentmem_c.h

# Default target
all: $(STATIC_LIB) $(SHARED_LIB)

# Static library
$(STATIC_LIB): $(OBJECTS)
	ar rcs $@ $^

# Shared library
$(SHARED_LIB): $(OBJECTS)
	$(CC) $(LDFLAGS) -o $@ $^

# Object files
%.o: %.c $(HEADERS)
	$(CC) $(CFLAGS) -c $< -o $@

# Clean
clean:
	rm -f $(OBJECTS) $(STATIC_LIB) $(SHARED_LIB)

# Install (optional)
install: $(STATIC_LIB) $(SHARED_LIB)
	mkdir -p /usr/local/lib
	mkdir -p /usr/local/include
	cp $(STATIC_LIB) $(SHARED_LIB) /usr/local/lib/
	cp $(HEADERS) /usr/local/include/

# Test compilation
test: $(STATIC_LIB)
	$(CC) -o test_agentmem test_agentmem.c -L. -lagentmem_c

.PHONY: all clean install test
