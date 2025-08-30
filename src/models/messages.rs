enum NotificationType {
    DriverAssigned,        // "Your driver Kwame is coming!"
    PackagePickedUp,       // "Your package has been collected"
    DriverNearby,          // "Your driver is 5min away!"  
    DeliveryCompleted,     // "Package delivered successfully!"
    RideStatusUpdate,      // "Status changed to In Progress"
    PaymentConfirmed,      // "Payment received via Mobile Money"
    GhanaPromotional,      // "Weekend discount for Accra deliveries!"
}

// English + Local language support
struct NotificationMessage {
    english: String,
    twi: Option<String>,     // For Akan speakers
    ga: Option<String>,      // For Ga speakers
    data: serde_json::Value, // App-specific data
}