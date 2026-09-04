//! Static definition sets for demo seeding.

pub const CATEGORIAS_DATA: [(&str, &str, &str, Option<usize>); 8] = [
    ("Materiales Eléctricos", "#3B82F6", "package", None),
    ("Cables y Conductores", "#3B82F6", "layers", Some(0)),
    ("Tableros y Protecciones", "#3B82F6", "shield", Some(0)),
    ("Herramientas y Equipos", "#F59E0B", "wrench", None),
    ("Servicios y Fletes", "#10B981", "briefcase", None),
    ("Impuestos y Tasas", "#EF4444", "receipt", None),
    ("Viáticos y Combustible", "#06B6D4", "truck", None),
    ("Gastos Administrativos", "#8B5CF6", "building", None),
];

pub const CUSTOM_TIPOS: [(&str, bool); 3] = [
    ("Venta de chatarra / sobrantes", true),
    ("Alquiler de andamios y equipos", true),
    ("Honorarios asesoría técnica", false),
];

pub const EMPLEADOS_DATA: [(&str, &str, &str, i64, i64, &str, &str); 5] = [
    ("Ricardo Darín", "20.123.456", "Operario Electricista", 4_500_000_000_i64, 450_000_000_i64, "1145678901", "ricardo.darin@proyecto.com"),
    ("Guillermo Francella", "22.345.678", "Capataz de Proyecto", 5_500_000_000_i64, 550_000_000_i64, "1145678902", "guillermo.francella@proyecto.com"),
    ("Natalia Oreiro", "25.678.901", "Técnica Instaladora", 4_800_000_000_i64, 480_000_000_i64, "1145678903", "natalia.oreiro@proyecto.com"),
    ("Diego Peretti", "18.901.234", "Ayudante Práctico", 3_800_000_000_i64, 380_000_000_i64, "1145678904", "diego.peretti@proyecto.com"),
    ("Érica Rivas", "27.234.567", "Administrativa de Proyecto", 4_200_000_000_i64, 420_000_000_i64, "1145678905", "erica.rivas@proyecto.com"),
];

pub const CLIENTES_DATA: [(&str, &str, &str, &str, &str, &str); 4] = [
    ("Constructora del Plata S.A.", "30-71234567-9", "Av. del Libertador 1234, CABA", "011-4567-8900", "info@constructoradelplata.com", "Responsable Inscripto"),
    ("Desarrollos Urbanos SRL", "30-79876543-1", "San Martín 567, Piso 4, Rosario", "0341-423-4567", "administracion@desarrollosurbanos.com", "Responsable Inscripto"),
    ("Consorcio Torre Alvear", "30-65432109-8", "Av. Alvear 1890, CABA", "011-4812-3456", "consorcio@torrealvear.com", "Consumidor Final"),
    ("Juan Carlos Pérez", "20-28123456-3", "Belgrano 432, San Isidro", "011-15-5432-1098", "jcperez@gmail.com", "Consumidor Final"),
];

pub const FERIADOS_DATA: [(&str, &str, &str); 9] = [
    ("2025-01-01", "Año Nuevo", "Inamovible"),
    ("2025-03-03", "Carnaval", "Inamovible"),
    ("2025-03-04", "Carnaval", "Inamovible"),
    ("2025-03-24", "Día Nacional de la Memoria por la Verdad y la Justicia", "Inamovible"),
    ("2025-04-02", "Día del Veterano y de los Caídos en la Guerra de Malvinas", "Inamovible"),
    ("2025-05-01", "Día del Trabajador", "Inamovible"),
    ("2025-05-25", "Día de la Revolución de Mayo", "Inamovible"),
    ("2025-07-09", "Día de la Independencia", "Inamovible"),
    ("2025-12-25", "Navidad", "Inamovible"),
];
