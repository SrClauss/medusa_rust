use crate::api::{store::currencies as store_currencies, admin::currencies as admin_currencies};

router
    .route("/store/currencies", get(store_currencies::list_currencies))
    .route("/admin/currencies", get(admin_currencies::list_admin_currencies).post(admin_currencies::create_currency))
    .route("/admin/currencies/:code", delete(admin_currencies::delete_currency));