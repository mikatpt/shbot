# Development Notes

## Project Design

- Slack bot listens for `DELIVER` and `REQUEST_WORK` events.
- On event, HTTP Server is sent a JSON info packet.

- HTTP Server (Axum) 
    - DELIVER
        - body: { user }
        - Retrieve movies
- Database server
    - Postgres

- Roles: [ AE, Editor, Sound, Finish ]

## Student facing reqs:

Requirements:
    - Priority queue of movies (HIGH LOW priority (only two priorities))
    - DELIVER button
        - When student finishes work, adds the movie to the available work queue.
        - Each time DELIVER is clicked, also check the waiting queue for waiting students
        - If there are waiting students, give them a movie from the work queue
    - REQUEST_WORK button
        - Polls from the movie queue and assigns movie to student.
        - If no movies are available, put student in WAIT queue - they will be assigned when next DELIVER 


Teacher facing side:
    - Show status of every movie

## Data layer
- [Diagram](https://sqldbm.com/Project/Dashboard/All/)

students
    - id
    - email
    - FK films_worked_on
    - FK prod_roles_worked

films
    - id
    - name
    - FK prod_roles

roles
    - ae
    - editor
    - sound
    - finish

student_films
    - FK student_id
    - FK film_id

student_roles
    - FK student_id
    - FK role_id
