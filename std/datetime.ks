export class DateTime {

    func initialize() {
        this.value = clock();
        this.date = this.date()
    }

    func timestamp() {
        return this.value;
    }

    // ========================================================
    // Calendar
    // ========================================================

    func isLeapYear(year) {

        if year % 400 == 0 {
            return true;
        }

        if year % 100 == 0 {
            return false;
        }

        return year % 4 == 0;
    }

    func daysInYear(year) {

        if this.isLeapYear(year) {
            return 366;
        }

        return 365;
    }

    func daysInMonth(year, month) {

        match month {

            1 => return 31;

            2 => {
                if this.isLeapYear(year) {
                    return 29;
                }

                return 28;
            }

            3 => return 31;

            4 => return 30;

            5 => return 31;

            6 => return 30;

            7 => return 31;

            8 => return 31;

            9 => return 30;

            10 => return 31;

            11 => return 30;

            12 => return 31;

            _ => return 0;
        }
    }

    // ========================================================
    // Year
    // ========================================================

    func year() {

        let days = int(this.value / 86400);
        let year = 1970;

        while days >= this.daysInYear(year) {
            days = days - this.daysInYear(year);
            year = year + 1;
        }

        return year;
    }

    // ========================================================
    // Month
    // ========================================================

    func month() {

        let days = int(this.value / 86400);
        let year = 1970;

        while days >= this.daysInYear(year) {
            days = days - this.daysInYear(year);
            year = year + 1;
        }

        let month = 1;

        while days >= this.daysInMonth(year, month) {
            days = days - this.daysInMonth(year, month);
            month = month + 1;
        }

        return month;
    }

    // ========================================================
    // Day
    // ========================================================

    func day() {

        let days = int(this.value / 86400);
        let year = 1970;

        while days >= this.daysInYear(year) {
            days = days - this.daysInYear(year);
            year = year + 1;
        }

        let month = 1;

        while days >= this.daysInMonth(year, month) {
            days = days - this.daysInMonth(year, month);
            month = month + 1;
        }

        return days + 1;
    }

    // ========================================================
    // Hour
    // ========================================================

    func hour() {
        let seconds = this.value;
        return int((seconds % 86400) / 3600);
    }

    // ========================================================
    // Minute
    // ========================================================

    func minute() {
        let seconds =  this.value;
        return int((seconds % 3600) / 60);
    }

    // ========================================================
    // Second
    // ========================================================

    func second() {
        let seconds = int(this.value);
        return (seconds % 60);
    }

    // ========================================================
    // Date
    // ========================================================

    func date() {

        return format("{}-{}-{}", this.year(),this.month(),this.day());
    }

    // ========================================================
    // Time
    // ========================================================

    func time() {
        return format("{}:{}:{}", this.hour(),this.minute(),this.second());
    }

    // ========================================================
    // String
    // ========================================================

    func toString() {
        return format("{} {}", this.date(),this.time());
    }

    // ========================================================
    // Arithmetic
    // ========================================================

    func addSeconds(value) {

        let result = new DateTime();

        result.value = this.value + value;

        return result;
    }

    func addMinutes(value) {

        let result = new DateTime();

        result.value = this.value + value * 60;

        return result;
    }

    func addHours(value) {

        let result = new DateTime();

        result.value = this.value + value * 3600;

        return result;
    }

    func addDays(value) {

        let result = new DateTime();

        result.value = this.value + value * 86400;

        return result;
    }
}
