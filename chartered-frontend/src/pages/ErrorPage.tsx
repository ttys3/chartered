import React = require("react");

export default function ErrorPage({ message }: { message: string }) {
  return (
    <div className="bg-primary min-vh-100 d-flex justify-content-center align-items-center">
      <div className="alert alert-danger" role="alert">
        {message}
      </div>
    </div>
  );
}
