// use surrealdb::engine::local::{Db, Mem};
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use tracing::info;
use crate::auth::hash_password;
use crate::models::*;

pub type AppDb = Surreal<Client>;

// pub async fn init_db() -> Result<AppDb, Box<dyn std::error::Error>> {
//     let db = Surreal::new::<Mem>(()).await?;
//     db.use_ns("academic").use_db("item_analysis").await?;
//     info!("Initialized SurrealDB in-memory database");

//     seed_data(&db).await?;
//     Ok(db)
// }

pub async fn init_db() -> Result<AppDb, Box<dyn std::error::Error>> {
    // Change "127.0.0.1:8003" to "127.0.0.1:8000" below:
    let mut db_url = std::env::var("SURREALDB_URL").unwrap_or_else(|_| "127.0.0.1:8000".to_string());
    let db_ns = std::env::var("SURREALDB_NS").unwrap_or_else(|_| "academic".to_string());
    let db_name = std::env::var("SURREALDB_DB").unwrap_or_else(|_| "item_analysis".to_string());

    // Clean up scheme prefixes if present, as Surreal::new::<Ws> expects "host:port" or "host:port/rpc"
    db_url = db_url.trim_start_matches("ws://")
        .trim_start_matches("wss://")
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .to_string();

    let db = Surreal::new::<Ws>(db_url.as_str()).await?;

    if let (Ok(username), Ok(password)) = (std::env::var("SURREALDB_USER"), std::env::var("SURREALDB_PASS")) {
        if !username.trim().is_empty() && !password.trim().is_empty() {
            db.signin(surrealdb::opt::auth::Root {
                username,
                password,
            })
            .await?;
        }
    }

    db.use_ns(db_ns.as_str())
        .use_db(db_name.as_str())
        .await?;

    // SurrealDB 3 databases may be strict, so create the application's
    // schemaless tables before the initial seed queries run.
    db.query(
        "DEFINE TABLE IF NOT EXISTS settings SCHEMALESS; \
         DEFINE TABLE IF NOT EXISTS user SCHEMALESS; \
         DEFINE TABLE IF NOT EXISTS assessment SCHEMALESS; \
         DEFINE TABLE IF NOT EXISTS question SCHEMALESS;",
    )
    .await?;

    info!("Connected to SurrealDB at {} (ns: {}, db: {})", db_url, db_ns, db_name);

    seed_data(&db).await?;

    Ok(db)
}


async fn seed_data(db: &AppDb) -> Result<(), Box<dyn std::error::Error>> {
    // Check if settings exist, if not seed settings
    let settings: Vec<SystemSettings> = db.query("SELECT * FROM settings").await?.take(0)?;
    if settings.is_empty() {
        let default_settings = SystemSettings {
            id: None,
            upper_lower_percentage: 27.0,
            thresholds: DiscriminationThresholds {
                excellent: 0.40,
                good: 0.30,
                fair: 0.20,
                poor: 0.00,
            },
            schools: vec![
                "School of Science & Technology".to_string(),
                "School of Business & Economics".to_string(),
                "School of Engineering & Applied Sciences".to_string(),
                "Faculty of Education".to_string(),
                "School of Medicine & Health Sciences".to_string(),
            ],
            assessment_types: vec![
                "Test".to_string(),
                "UE".to_string(),
                "Assignment".to_string(),
                "Examination".to_string(),
                "Quiz".to_string(),
                "Midterm".to_string(),
                "Final".to_string(),
            ],
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        let _: Option<SystemSettings> = db.create(("settings", "default")).content(default_settings).await?;
        info!("Seeded system settings");
    }

    // Check if superadmin exists, if not seed users
    let users: Vec<User> = db.query("SELECT * FROM user").await?.take(0)?;
    if users.is_empty() {
        let superadmin_hash = hash_password("SuperAdmin123!").unwrap();
        let admin_hash = hash_password("Admin123!").unwrap();
        let lecturer_hash = hash_password("Lecturer123!").unwrap();

        let superadmin = User {
            id: None,
            first_name: "Super".to_string(),
            middle_name: Some("System".to_string()),
            surname: "Admin".to_string(),
            school: "System Administration".to_string(),
            profession: "System Director".to_string(),
            specialization: "Educational Tech & Administration".to_string(),
            email: "superadmin@system.com".to_string(),
            password_hash: superadmin_hash,
            role: UserRole::Superadmin,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let admin = User {
            id: None,
            first_name: "Academic".to_string(),
            middle_name: Some("Quality".to_string()),
            surname: "Manager".to_string(),
            school: "School of Science & Technology".to_string(),
            profession: "Assessment Admin".to_string(),
            specialization: "Psychometrics & Evaluation".to_string(),
            email: "admin@system.com".to_string(),
            password_hash: admin_hash,
            role: UserRole::Admin,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let lecturer = User {
            id: None,
            first_name: "Dr. Jane".to_string(),
            middle_name: Some("M.".to_string()),
            surname: "Doe".to_string(),
            school: "School of Science & Technology".to_string(),
            profession: "Senior Lecturer".to_string(),
            specialization: "Computer Science & Data Analysis".to_string(),
            email: "lecturer@university.edu".to_string(),
            password_hash: lecturer_hash,
            role: UserRole::Lecturer,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let _: Option<User> = db.create(("user", "user_superadmin")).content(superadmin).await?;
        let _: Option<User> = db.create(("user", "user_admin")).content(admin).await?;
        let _: Option<User> = db.create(("user", "user_lecturer")).content(lecturer).await?;

        info!("Seeded default users (superadmin@system.com, admin@system.com, lecturer@university.edu)");

        // Seed a sample assessment for demonstration
        let sample_assessment_id = "asm_sample_1".to_string();
        let num_students = 100;
        let percentage = 27.0;
        let group_size = (num_students as f64 * percentage / 100.0).round() as usize; // 27
        let middle_size = num_students - (2 * group_size); // 46

        let sample_asm = Assessment {
            id: None,
            user_id: "user_lecturer".to_string(),
            lecturer_name: "Dr. Jane M. Doe".to_string(),
            school: "School of Science & Technology".to_string(),
            subject: "CS301 - Algorithms & Data Structures".to_string(),
            assessment_type: "Examination".to_string(),
            num_students,
            title: "End of Semester Final Examination".to_string(),
            date: "2026-06-15".to_string(),
            upper_percentage: percentage,
            upper_group_size: group_size,
            lower_group_size: group_size,
            middle_group_size: middle_size,
            status: AssessmentStatus::Submitted,
            admin_feedback: Some("Assessment details and sample items complete. Ready for verification review.".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        let _: Option<Assessment> = db.create(("assessment", "asm_sample_1")).content(sample_asm).await?;

        // Seed 10 sample questions with diverse difficulty and discrimination indices
        let questions_data = vec![
            (1, "What is the time complexity of QuickSort average case?", 25, 5),
            (2, "Explain binary search tree balancing.", 24, 8),
            (3, "Which data structure uses LIFO principle?", 27, 26),
            (4, "Analyze Dijkstra's shortest path algorithm.", 20, 10),
            (5, "Identify NP-complete problem characteristics.", 12, 11),
            (6, "Implement dynamic programming matrix multiplication.", 18, 5),
            (7, "Differentiate between BFS and DFS traversal.", 23, 12),
            (8, "Calculate graph chromatic number.", 8, 14), // Negative discrimination sample
            (9, "Define asymptotic notation Big-O.", 26, 18),
            (10, "Hash collision resolution strategies.", 22, 9),
        ];

        for (q_num, prompt, upper_c, lower_c) in questions_data {
            let u_c = upper_c.min(group_size);
            let u_w = group_size - u_c;
            let l_c = lower_c.min(group_size);
            let l_w = group_size - l_c;

            let diff_idx = (u_c + l_c) as f64 / (2 * group_size) as f64;
            let disc_idx = (u_c as f64 - l_c as f64) / group_size as f64;

            let p_clamped = diff_idx.clamp(0.001, 0.999);
            let rasch_diff = ((1.0 - p_clamped) / p_clamped).ln();
            let rasch_difficulty = (rasch_diff * 1000.0).round() / 1000.0;

            let interpretation = if disc_idx >= 0.4 {
                "Very Good Item"
            } else if disc_idx >= 0.3 {
                "Good Item"
            } else if disc_idx >= 0.2 {
                "Acceptable Item"
            } else if disc_idx >= 0.0 {
                "Poor Item"
            } else {
                "Very poor Item"
            };

            let decision = if disc_idx >= 0.4 {
                "retain (Very good discrimination ability)"
            } else if disc_idx >= 0.3 {
                "retain/revise (Good item, minor adjustments if necessary)"
            } else if disc_idx >= 0.2 {
                "revise (Acceptable item, needs revision or distractor review)"
            } else if disc_idx >= 0.0 {
                "revise (Poor item, major overhaul required)"
            } else {
                "remove (Very poor item, negative discrimination or incorrect key)"
            };

            let question = QuestionItem {
                id: None,
                assessment_id: sample_assessment_id.clone(),
                question_number: q_num,
                prompt: Some(prompt.to_string()),
                upper_correct: u_c,
                upper_wrong: u_w,
                lower_correct: l_c,
                lower_wrong: l_w,
                difficulty_index: (diff_idx * 1000.0).round() / 1000.0,
                discrimination_index: (disc_idx * 1000.0).round() / 1000.0,
                rasch_difficulty,
                interpretation: interpretation.to_string(),
                decision: decision.to_string(),
                classification: interpretation.to_string(),
                recommendation: decision.to_string(),
            };

            let q_key = format!("q_{}_{}", sample_assessment_id, q_num);
            let _: Option<QuestionItem> = db.create(("question", q_key)).content(question).await?;
        }

        info!("Seeded sample assessment with 10 question items");
    }

    Ok(())
}
