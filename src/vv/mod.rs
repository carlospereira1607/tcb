/**
 * Transport layer of the middleware service.
 */
pub mod communication;
/**
 * Middleware that ensures causal delivery.
 */
pub mod middleware;
/**
 * Common structs of the middleware service.
 */
pub mod structs;
/**
 * API and necessary state for the client to communicate with the middleware.
 */
pub mod version_vector;
