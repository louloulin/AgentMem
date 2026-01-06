package agentmem

import (
	"fmt"
)

// AgentMemError represents a base error from AgentMem API
type AgentMemError struct {
	Message    string
	StatusCode int
	Code       string
}

func (e *AgentMemError) Error() string {
	if e.Code != "" {
		return fmt.Sprintf("AgentMem error [%s]: %s (status: %d)", e.Code, e.Message, e.StatusCode)
	}
	return fmt.Sprintf("AgentMem error: %s (status: %d)", e.Message, e.StatusCode)
}

// AuthenticationError represents authentication failures
type AuthenticationError struct {
	*AgentMemError
}

// NewAuthenticationError creates a new authentication error
func NewAuthenticationError(message string) *AuthenticationError {
	if message == "" {
		message = "Authentication failed"
	}
	return &AuthenticationError{
		AgentMemError: &AgentMemError{
			Message:    message,
			StatusCode: 401,
			Code:       "AUTHENTICATION_ERROR",
		},
	}
}

// ValidationError represents request validation failures
type ValidationError struct {
	*AgentMemError
}

// NewValidationError creates a new validation error
func NewValidationError(message string) *ValidationError {
	if message == "" {
		message = "Request validation failed"
	}
	return &ValidationError{
		AgentMemError: &AgentMemError{
			Message:    message,
			StatusCode: 400,
			Code:       "VALIDATION_ERROR",
		},
	}
}

// NetworkError represents network communication errors
type NetworkError struct {
	*AgentMemError
}

// NewNetworkError creates a new network error
func NewNetworkError(message string) *NetworkError {
	if message == "" {
		message = "Network error"
	}
	return &NetworkError{
		AgentMemError: &AgentMemError{
			Message:    message,
			StatusCode: 0,
			Code:       "NETWORK_ERROR",
		},
	}
}

// NotFoundError represents resource not found errors
type NotFoundError struct {
	*AgentMemError
}

// NewNotFoundError creates a new not found error
func NewNotFoundError(message string) *NotFoundError {
	if message == "" {
		message = "Resource not found"
	}
	return &NotFoundError{
		AgentMemError: &AgentMemError{
			Message:    message,
			StatusCode: 404,
			Code:       "NOT_FOUND_ERROR",
		},
	}
}

// RateLimitError represents rate limiting errors
type RateLimitError struct {
	*AgentMemError
}

// NewRateLimitError creates a new rate limit error
func NewRateLimitError(message string) *RateLimitError {
	if message == "" {
		message = "Rate limit exceeded"
	}
	return &RateLimitError{
		AgentMemError: &AgentMemError{
			Message:    message,
			StatusCode: 429,
			Code:       "RATE_LIMIT_ERROR",
		},
	}
}

// ServerError represents server-side errors
type ServerError struct {
	*AgentMemError
}

// NewServerError creates a new server error
func NewServerError(message string) *ServerError {
	if message == "" {
		message = "Server error"
	}
	return &ServerError{
		AgentMemError: &AgentMemError{
			Message:    message,
			StatusCode: 500,
			Code:       "SERVER_ERROR",
		},
	}
}

// handleHTTPError converts HTTP status codes to appropriate error types
func handleHTTPError(statusCode int, message string) error {
	switch statusCode {
	case 401:
		return NewAuthenticationError(message)
	case 400:
		return NewValidationError(message)
	case 404:
		return NewNotFoundError(message)
	case 429:
		return NewRateLimitError(message)
	case 500, 502, 503, 504:
		return NewServerError(message)
	default:
		return &AgentMemError{
			Message:    message,
			StatusCode: statusCode,
			Code:       "UNKNOWN_ERROR",
		}
	}
}
