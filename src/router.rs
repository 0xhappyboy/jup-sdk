/// An abstract module for Jupiter routing.
use crate::types::QuoteResponse;

/// Route analysis result for comparison and selection of optimal routes
#[derive(Debug, Clone)]
pub struct RouteAnalysis {
    pub best_route: QuoteResponse,
    pub alternative_routes: Vec<QuoteResponse>,
    pub estimated_time: f64,
    pub confidence_score: f64,
}

impl RouteAnalysis {
    /// Creates a new RouteAnalysis with the specified best route
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::types::QuoteResponse;
    /// use crate::router::RouteAnalysis;
    ///
    /// let quote_response = QuoteResponse::default();
    /// let analysis = RouteAnalysis::new(quote_response);
    /// ```
    pub fn new(best_route: QuoteResponse) -> Self {
        Self {
            best_route,
            alternative_routes: Vec::new(),
            estimated_time: 0.0,
            confidence_score: 1.0,
        }
    }
}

/// Route optimizer for selecting and scoring trading routes
pub struct RouteOptimizer;

impl RouteOptimizer {
    /// Selects the best route from a list of routes based on weighted criteria
    ///
    /// # Arguments
    ///
    /// * `routes` - Slice of QuoteResponse to evaluate
    /// * `weights` - RouteWeights configuration for scoring
    ///
    /// # Returns
    ///
    /// Optional reference to the best QuoteResponse
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::router::{RouteOptimizer, RouteWeights};
    /// use crate::types::QuoteResponse;
    ///
    /// let routes = vec![QuoteResponse::default(), QuoteResponse::default()];
    /// let weights = RouteWeights::default();
    /// let best_route = RouteOptimizer::select_best_route(&routes, &weights);
    /// ```
    pub fn select_best_route<'a>(
        routes: &'a [QuoteResponse],
        weights: &'a RouteWeights,
    ) -> Option<&'a QuoteResponse> {
        routes.iter().max_by(|a, b| {
            let score_a = Self::cal_route_score(a, weights);
            let score_b = Self::cal_route_score(b, weights);
            score_a
                .partial_cmp(&score_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Calculates a comprehensive score for a route based on multiple factors
    ///
    /// # Arguments
    ///
    /// * `route` - The QuoteResponse to score
    /// * `weights` - Weight configuration for different scoring factors
    ///
    /// # Returns
    ///
    /// Computed score as f64 where higher values indicate better routes
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::router::{RouteOptimizer, RouteWeights};
    /// use crate::types::QuoteResponse;
    ///
    /// let route = QuoteResponse::default();
    /// let weights = RouteWeights::default();
    /// let score = RouteOptimizer::cal_route_score(&route, &weights);
    /// ```
    fn cal_route_score(route: &QuoteResponse, weights: &RouteWeights) -> f64 {
        let mut score = 0.0;
        if let Ok(price_impact) = route.price_impact_pct.parse::<f64>() {
            score += (100.0 - price_impact.max(0.0)) * weights.price_impact;
        }
        score += (1000.0 - route.time_taken.max(0.0)) * weights.execution_speed;
        let complexity = 1.0 / (route.route_plan.len() as f64).max(1.0);
        score += complexity * weights.simplicity;
        score
    }
}

/// Weight configuration for route scoring criteria
#[derive(Debug, Clone)]
pub struct RouteWeights {
    pub price_impact: f64,
    pub execution_speed: f64,
    pub simplicity: f64,
}

impl Default for RouteWeights {
    /// Creates default RouteWeights with balanced priorities
    ///
    /// Default weights prioritize price impact (0.6), followed by execution speed (0.3)
    /// and simplicity (0.1)
    fn default() -> Self {
        Self {
            price_impact: 0.6,
            execution_speed: 0.3,
            simplicity: 0.1,
        }
    }
}
