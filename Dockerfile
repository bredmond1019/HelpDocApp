FROM ubuntu:20.04

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive

# Install necessary packages including PostgreSQL
RUN apt-get update && apt-get install -y \
  curl \
  build-essential \
  nodejs \
  npm \
  postgresql \
  postgresql-contrib \
  && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set working directory
WORKDIR /app

# Create React app
RUN npx create-react-app frontend --template typescript

# Create Rust project
RUN cargo new backend

# Install frontend dependencies
WORKDIR /app/frontend
RUN npm install react-router-dom @types/react-router-dom

# Install backend dependencies
WORKDIR /app/backend
RUN cargo add actix-web tokio serde serde_json reqwest dotenv

# Create project structure
WORKDIR /app
RUN mkdir -p frontend/src/{components,pages,services} \
  && mkdir -p backend/src/{routes,models,services,db} \
  && touch frontend/src/{App.tsx,components/{ChatInterface.tsx,Message.tsx},pages/ChatPage.tsx,services/websocket.ts} \
  && touch backend/src/{main.rs,routes/chat.rs,models/message.rs,services/{ai_model.rs,api_integration.rs},db/connection.rs}

# Create PostgreSQL database
USER postgres
RUN /etc/init.d/postgresql start && \
  psql --command "CREATE DATABASE ai_chatbot;" && \
  /etc/init.d/postgresql stop

# Switch back to root user
USER root

# Create .env file
RUN echo "DATABASE_URL=postgres://postgres@localhost/ai_chatbot" > backend/.env \
  && echo "API_KEY=your_api_key_here" >> backend/.env

# Expose ports
EXPOSE 3000 8080 5432

# Start command (you might want to adjust this based on your needs)
CMD service postgresql start && sh -c "cd frontend && npm start & cd backend && cargo run"
